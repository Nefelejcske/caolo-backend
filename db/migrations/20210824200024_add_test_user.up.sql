-- username=asd
-- pw=asdasdasd
INSERT INTO public.user_account ( username, pw, salt )
VALUES ('asd', '$2b$12$U.VrU/gw9tfi59KasXMX0.S8wnTLBSWXfPz49INCT0CYRKOnxiaPm' , 'ApIQwtnvds')
ON CONFLICT DO NOTHING;
