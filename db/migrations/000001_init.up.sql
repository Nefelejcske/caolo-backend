CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: on_world_ouput_insert(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.on_world_ouput_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM world_hot
    WHERE
        id NOT IN (
            SELECT foo.id
            FROM (
                SELECT id
                FROM world_hot
                ORDER BY created DESC
                -- TODO this should consider the queen_tag as well...
                LIMIT 200
            ) foo
        );

    RETURN NULL;
END;
$$;


ALTER FUNCTION public.on_world_ouput_insert() OWNER TO postgres;

--
-- Name: set_updated_col(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.set_updated_col() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated = now();
    RETURN NEW;   
END;
$$;


ALTER FUNCTION public.set_updated_col() OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: user_account; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.user_account (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    display_name character varying,
    email character varying,
    username character varying NOT NULL,
    pw character varying,
    salt character varying,
    token character varying,
    created timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone DEFAULT now() NOT NULL,
    email_verified boolean DEFAULT false NOT NULL
);


ALTER TABLE public.user_account OWNER TO postgres;

--
-- Name: user_script; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.user_script (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    owner_id uuid,
    program json NOT NULL,
    created timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone DEFAULT now() NOT NULL,
    name character varying NOT NULL
);


ALTER TABLE public.user_script OWNER TO postgres;

--
-- Name: user_account email_is_unique; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_account
    ADD CONSTRAINT email_is_unique UNIQUE (email);


--
-- Name: user_script name_owner_id_unique; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_script
    ADD CONSTRAINT name_owner_id_unique UNIQUE (name, owner_id);


--
-- Name: user_account user_account_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_account
    ADD CONSTRAINT user_account_pkey PRIMARY KEY (id);


--
-- Name: user_script user_script_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_script
    ADD CONSTRAINT user_script_pkey PRIMARY KEY (id);


--
-- Name: user_account username_is_unique; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_account
    ADD CONSTRAINT username_is_unique UNIQUE (username);


--
-- Name: user_account user_account_updated; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER user_account_updated AFTER UPDATE ON public.user_account FOR EACH ROW EXECUTE FUNCTION public.set_updated_col();


--
-- Name: user_script user_script_updated; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER user_script_updated AFTER UPDATE ON public.user_script FOR EACH ROW EXECUTE FUNCTION public.set_updated_col();


--
-- PostgreSQL database dump complete
--
