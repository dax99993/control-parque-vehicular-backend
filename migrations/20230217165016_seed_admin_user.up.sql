-- Add up migration script here
INSERT INTO usuarios (usuario_id, nombres, apellidos, email, password_hash, verificado, rol)
VALUES (
    'af162cde-9de7-4eae-a446-9e2a62608e2c',
    'admin',
    'user',
    'adminuser@email.com',
    '$argon2id$v=19$m=15000,t=2,p=1$EaX3Qdh6NxifVkYF8DWWFQ$Wd4fxMx9drCUnMdwMtO6g+YGnE8pOjXTkzKMKL7F0Jk',
    true,
    'admin'
);

-- Maybe add the sigunup token to table even if verifing the email is not used?
