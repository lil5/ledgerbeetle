CREATE TABLE
  accounts (
    id BIGSERIAL PRIMARY KEY,
    "name" VARCHAR NOT NULL,
    tb_id VARCHAR(31) NOT NULL
  );