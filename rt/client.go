package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"time"

	"github.com/gorilla/websocket"
	"go.uber.org/zap"
)

// Single client handler
type client struct {
	logger      *zap.Logger
	conn        *websocket.Conn
	hub         *GameStateHub
	roomIds     []RoomId
	entities    chan *RoomState
	onNewRoomId chan RoomId
}

func NewClient(logger *zap.Logger, conn *websocket.Conn, hub *GameStateHub) client {
	return client{
		logger:      logger,
		conn:        conn,
		hub:         hub,
		roomIds:     make([]RoomId, 0, 100),
		entities:    make(chan *RoomState, 100),
		onNewRoomId: make(chan RoomId, 100),
	}
}

type InputMsg struct {
	Ty      string   `json:"ty"`
	RoomId  RoomId   `json:"room_id,omitempty"`
	RoomIds []RoomId `json:"room_ids,omitempty"`
}

func FindRoomIdIndex(arr []RoomId, key RoomId) int {
	for i := range arr {
		if arr[i] == key {
			return i
		}
	}
	return -1
}

/// Might move the last item so delete doesn't trigger a reordering of elements
func RemoveRoomId(arr []RoomId, key RoomId) []RoomId {
	index := FindRoomIdIndex(arr, key)
	if index < 0 {
		return arr
	}

	// swap with the last
	arr[index] = arr[len(arr)-1]

	return arr[:len(arr)-1]
}

func (c *client) readPump() {
	defer func() {
		c.hub.unregister <- c
		c.conn.Close()
	}()
	c.conn.SetReadLimit(10 * 1024)
	c.conn.SetReadDeadline(time.Now().Add(60 * time.Second))
	c.conn.SetPongHandler(func(string) error { c.conn.SetReadDeadline(time.Now().Add(60 * time.Second)); return nil })
	for {
		_, msg, err := c.conn.ReadMessage()
		if err != nil {
			c.logger.Info("Client going away", zap.Error(err))
			return
		}
		msg = bytes.TrimSpace(bytes.Replace(msg, []byte{'\n'}, []byte{' '}, -1))
		var pl InputMsg
		err = json.Unmarshal(msg, &pl)
		if err != nil {
			c.logger.Warn("Invalid message", zap.Error(err))
			return
		}
		c.logger.Debug("Incoming message", zap.Any("ty", pl.Ty))
		switch pl.Ty {
		case "room_ids":
			if len(c.roomIds)+len(pl.RoomIds) > 100 {
				c.logger.Debug("Client is listening to too many roomIds")
				continue
			}
			c.logger.Debug("Client subscribed to", zap.Any("roomIds", pl.RoomIds))
			c.roomIds = append(c.roomIds, pl.RoomIds...)

			for i := range pl.RoomIds {
				id := pl.RoomIds[i]
				c.onNewRoomId <- id
			}
		case "room_id":
			if len(c.roomIds) >= 100 {
				c.logger.Debug("Client is listening to too many roomIds")
				continue
			}
			c.logger.Debug("Client subscribed to", zap.Any("roomId", pl.RoomId))
			c.roomIds = append(c.roomIds, pl.RoomId)
			c.onNewRoomId <- pl.RoomId
		case "unsubscribe_room_id":
			c.logger.Debug("Client unsubscribed from", zap.Any("roomId", pl.RoomId))
			c.roomIds = RemoveRoomId(c.roomIds, pl.RoomId)
		case "unsubscribe_room_ids":
			c.logger.Debug("Client unsubscribed from", zap.Any("roomIds", pl.RoomIds))
			for i := range pl.RoomIds {
				c.roomIds = RemoveRoomId(c.roomIds, pl.RoomIds[i])
			}
		case "clear_room_ids":
			c.logger.Debug("Client cleared their room subs")
			c.roomIds = []RoomId{}
		default:
			c.logger.Warn("Unhandled msg type", zap.Any("payload", pl))
		}
	}
}

type Response struct {
	Ty      string      `json:"ty"`
	Payload interface{} `json:"payload"`
}

func sendJson(conn *websocket.Conn, ty string, payload interface{}) error {
	response := Response{
		Ty:      ty,
		Payload: payload,
	}
	pl, err := json.Marshal(response)
	if err != nil {
		return err
	}
	w, err := conn.NextWriter(websocket.TextMessage)
	if err != nil {
		return err
	}
	w.Write(pl)

	return nil
}

func (c *client) writePump() {
	ticker := time.NewTicker(50 * time.Second)
	defer func() {
		ticker.Stop()
		c.conn.Close()
	}()

	for {
		select {
		case roomId, ok := <-c.onNewRoomId:
			if !ok {
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}
			terrain := c.hub.Terrain[roomId]
			if terrain == nil {
				continue
			}
			err := sendJson(c.conn, "terrain", terrain)
			if err != nil {
				c.logger.Debug("Failed to send terrain", zap.Error(err))
				return
			}
			entities := c.hub.Entities[roomId]
			err = sendJson(c.conn, "entities", entities)
			if err != nil {
				c.logger.Debug("Failed to send initial entities", zap.Error(err))
				return
			}
		case entities, ok := <-c.entities:
            c.logger.Debug("Sending entities", zap.Any("Time", entities.Time), zap.Any("RoomId", entities.RoomId))
			c.conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if !ok {
				// hub closed this channel
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				c.logger.Debug("Hub closed the channel")
				return
			}
			err := sendJson(c.conn, "entities", entities)
			if err != nil {
				c.logger.Debug("Failed to send entities", zap.Error(err))
				return
			}
            c.logger.Debug("Sending entities done", zap.Any("Time", entities.Time), zap.Any("RoomId", entities.RoomId))
		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				c.logger.Debug("Failed to ping", zap.Error(err))
				return
			}
		}
	}
}

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

func ServeWs(logger *zap.Logger, hub *GameStateHub, w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		logger.Warn("Failed to upgrade ws connection", zap.Error(err))
	}
	client := NewClient(logger, conn, hub)
	hub.register <- &client

	logger.Debug("New client")

	go client.writePump()
	go client.readPump()
}
