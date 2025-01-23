ALTER TABLE whitelisted_addresses ADD COLUMN referral_code CHAR(32);
ALTER TABLE whitelisted_addresses ADD COLUMN referred_from VARCHAR(32);
ALTER TABLE whitelisted_addresses ADD UNIQUE INDEX referral_code_idx(referral_code);

CREATE TABLE IF NOT EXISTS referrals(
  refkey CHAR(128) NOT NULL,
  refvalue CHAR(32),
  PRIMARY KEY (refkey),
  INDEX ref_idx (refvalue)
);
