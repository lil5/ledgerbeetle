CREATE TABLE
  commodities (
    id SERIAL PRIMARY KEY,
    tb_ledger SERIAL NOT NULL,
    unit TEXT NOT NULL,
    decimal_place INT DEFAULT 0 NOT NULL
  );

ALTER TABLE accounts
ADD COLUMN commodities_id INT NOT NULL;

ALTER TABLE accounts
ADD CONSTRAINT fk_commodities FOREIGN KEY (commodities_id) REFERENCES commodities (id);

ALTER TABLE accounts
ADD CONSTRAINT uk_accounts_name_commodities UNIQUE (name, commodities_id);