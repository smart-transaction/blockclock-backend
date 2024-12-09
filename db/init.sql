-- Initial database setup. Run as root@

-- Create the database.
CREATE DATABASE IF NOT EXISTS timekeeper;
USE timekeeper;

-- Create the whitelist table.
CREATE TABLE IF NOT EXISTS whitelisted_addresses (
  address VARCHAR(255),
  avatar VARCHAR(255),
  PRIMARY KEY (address),
  UNIQUE_INDEX avatar_idx(avatar)
);

-- Create the user.
-- 1. Remove '%' user
--    if the server and mysql run on the same instance.
--    (still needed if run from two images)
CREATE USER IF NOT EXISTS 'server'@'localhost' IDENTIFIED BY 'secret_app';
CREATE USER IF NOT EXISTS 'server'@'%' IDENTIFIED BY 'secret_app';
CREATE USER IF NOT EXISTS 'importer'@'%' IDENTIFIED BY 'secret_importer';
SELECT User, Host FROM mysql.user;

-- Grant rights to the user.
GRANT ALL ON timekeeper.* TO 'server'@'localhost';
GRANT ALL ON timekeeper.* TO 'server'@'%';
GRANT SELECT ON timekeeper.* TO 'importer'@'%';  -- We don't make secret out of reports, so that's safe.
