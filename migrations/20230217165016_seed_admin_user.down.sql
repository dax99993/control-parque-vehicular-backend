-- Add down migration script here
DELETE FROM users WHERE id = 'af162cde-9de7-4eae-a446-9e2a62608e2c';
-- Maybe delete the sigunup token?
