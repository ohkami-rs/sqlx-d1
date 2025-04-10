-- Migration number: 0002 	 2025-04-10T16:03:08.577Z

ALTER TABLE users ADD COLUMN uuid TEXT; -- nullable for alter table
