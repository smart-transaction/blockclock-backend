-- Initial database setup. Run as root@

-- Create the database.
CREATE DATABASE IF NOT EXISTS timekeeper;
USE timekeeper;

-- Create the whitelist table.
CREATE TABLE IF NOT EXISTS whitelisted_addresses (
  address VARCHAR(255),
  avatar VARCHAR(255),
  referral_code CHAR(32),
  referred_from VARCHAR(32),
  PRIMARY KEY (address),
  UNIQUE INDEX avatar_idx(avatar),
  UNIQUE INDEX referral_code_idx(referral_code)
);

CREATE TABLE IF NOT EXISTS referrals(
  refkey CHAR(128) NOT NULL,
  refvalue CHAR(32),
  PRIMARY KEY (refkey),
  INDEX ref_idx (refvalue)
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
