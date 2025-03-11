UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'assets(:.*)', 'a\1')
WHERE
  "name" LIKE 'assets:%';

UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'liabilities(:.*)', 'l\1')
WHERE
  "name" LIKE 'liabilities:%';

UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'equity(:.*)', 'e\1')
WHERE
  "name" LIKE 'equity:%';

UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'revenues(:.*)', 'r\1')
WHERE
  "name" LIKE 'revenues:%';

UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'expenses(:.*)', 'x\1')
WHERE
  "name" LIKE 'expenses:%';