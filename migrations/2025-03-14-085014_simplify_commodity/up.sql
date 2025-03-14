CREATE TABLE
  commodities_new (
    id SERIAL PRIMARY KEY,
    unit TEXT NOT NULL,
    decimal_place INT DEFAULT 0 NOT NULL
  );

INSERT INTO
  commodities_new
SELECT
  c.tb_ledger AS id,
  c.unit AS unit,
  c.decimal_place AS decimal_place
FROM
  commodities c;

ALTER TABLE accounts
ADD COLUMN commodities_new_id INT;

UPDATE accounts
SET
  commodities_new_id = c.tb_ledger
FROM
  commodities c
WHERE
  c.id = commodities_id;

ALTER TABLE accounts
ALTER COLUMN commodities_new_id
SET NOT NULL;

ALTER TABLE accounts
DROP CONSTRAINT fk_commodities;

ALTER TABLE accounts
DROP CONSTRAINT uk_accounts_name_commodities;

ALTER TABLE accounts
DROP COLUMN commodities_id;

DROP TABLE commodities;

ALTER TABLE commodities_new
RENAME TO commodities;

ALTER TABLE accounts
RENAME COLUMN commodities_new_id TO commodities_id;

ALTER TABLE accounts
ADD CONSTRAINT fk_commodities FOREIGN KEY (commodities_id) REFERENCES commodities (id);

ALTER TABLE accounts
ADD CONSTRAINT uk_accounts_name_commodities UNIQUE ("name", commodities_id);

ALTER TABLE commodities
ADD CONSTRAINT uk_commodities_unit UNIQUE (unit);