CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner:
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';

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


CREATE TABLE public.user_script (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    owner_id uuid,
    program json NOT NULL,
    created timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone DEFAULT now() NOT NULL,
    name character varying NOT NULL
);


ALTER TABLE ONLY public.user_account
    ADD CONSTRAINT email_is_unique UNIQUE (email);


ALTER TABLE ONLY public.user_script
    ADD CONSTRAINT name_owner_id_unique UNIQUE (name, owner_id);



ALTER TABLE ONLY public.user_account
    ADD CONSTRAINT user_account_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.user_script
    ADD CONSTRAINT user_script_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.user_account
    ADD CONSTRAINT username_is_unique UNIQUE (username);



CREATE TRIGGER user_account_updated AFTER UPDATE ON public.user_account FOR EACH ROW EXECUTE FUNCTION public.set_updated_col();



CREATE TRIGGER user_script_updated AFTER UPDATE ON public.user_script FOR EACH ROW EXECUTE FUNCTION public.set_updated_col();
