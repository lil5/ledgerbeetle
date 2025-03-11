UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'a(:.*)', 'assets\1')
WHERE
  "name" LIKE 'a:%';

UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'l(:.*)', 'liabilities\1')
WHERE
  "name" LIKE 'l:%';

UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'e(:.*)', 'equity\1')
WHERE
  "name" LIKE 'e:%';

UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'r(:.*)', 'revenues\1')
WHERE
  "name" LIKE 'r:%';

UPDATE accounts
SET
  "name" = REGEXP_REPLACE ("name", 'x(:.*)', 'expenses\1')
WHERE
  "name" LIKE 'x:%';