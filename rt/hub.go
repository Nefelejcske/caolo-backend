package main

import (
	cao_world "github.com/caolo-game/cao-rt/cao_world_pb"
	"go.uber.org/zap"
)

type GameStateHub struct {
	logger   *zap.Logger
	Entities map[RoomId]RoomState
	Terrain  map[RoomId]*cao_world.RoomTerrain

	clients map[*client]bool

	// push new worldState to hub
	WorldState chan *cao_world.RoomEntities

	/// register new Clients
	register chan *client
	/// un-register new Clients
	unregister chan *client
}

type RoomState struct {
	Time       int64                  `json:"time"`
	RoomId     RoomId                 `json:"roomId"`
	Bots       []*cao_world.Bot       `json:"bots"`
	Structures []*cao_world.Structure `json:"structures"`
	Resources  []*cao_world.Resource  `json:"resources"`
}

func NewGameStateHub() *GameStateHub {
	return &GameStateHub{
		Entities:   map[RoomId]RoomState{},
		Terrain:    map[RoomId]*cao_world.RoomTerrain{},
		clients:    map[*client]bool{},
		WorldState: make(chan *cao_world.RoomEntities),
		register:   make(chan *client),
		unregister: make(chan *client),
	}
}

func (hub *GameStateHub) Run() {
	for {
		select {
		case newEntities := <-hub.WorldState:
			time := newEntities.WorldTime
			rid := newEntities.GetRoomId()
			roomId := RoomId{
				Q: rid.Q,
				R: rid.R,
			}

			var state RoomState
			if s, ok := hub.Entities[roomId]; ok {
				state = s
				state.Time = time
				state.RoomId = roomId
			} else {
				state = RoomState{
					Time:       time,
					RoomId:     roomId,
					Bots:       []*cao_world.Bot{},
					Structures: []*cao_world.Structure{},
					Resources:  []*cao_world.Resource{},
				}
			}
			state.Bots = newEntities.Bots
			state.Structures = newEntities.Structures
			state.Resources = newEntities.Resources

			hub.Entities[roomId] = state

			for client := range hub.clients {
				ind := FindRoomIdIndex(client.roomIds, roomId)
				if ind < 0 {
					continue
				}
				select {
				case client.entities <- &state:
				default:
					hub.logger.Info("Failed to send state to client, closing connection", zap.Reflect("client", client))
					delete(hub.clients, client)
					close(client.entities)
				}
			}
		case newClient := <-hub.register:
			hub.clients[newClient] = true
		case ex := <-hub.unregister:
			if _, ok := hub.clients[ex]; ok {
				delete(hub.clients, ex)
			}
		}
	}
}
