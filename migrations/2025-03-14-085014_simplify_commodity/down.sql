ALTER TABLE commodities
ADD COLUMN tb_ledger SERIAL;

UPDATE commodities
SET
  tb_ledger = id;

ALTER TABLE commodities
ALTER COLUMN tb_ledger
SET NOT NULL;