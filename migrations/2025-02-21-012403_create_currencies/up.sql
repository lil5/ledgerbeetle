CREATE TABLE
  currencies (
    id SERIAL PRIMARY KEY,
    tb_ledger SERIAL NOT NULL,
    unit TEXT NOT NULL
  );

ALTER TABLE accounts
ADD COLUMN currencies_id INT UNIQUE;

ALTER TABLE accounts
ADD CONSTRAINT fk_currencies FOREIGN KEY (currencies_id) REFERENCES currencies (id);