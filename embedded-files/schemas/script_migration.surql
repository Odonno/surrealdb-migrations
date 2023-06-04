DEFINE TABLE script_migration SCHEMAFULL;

DEFINE FIELD script_name ON script_migration TYPE string;
DEFINE FIELD executed_at ON script_migration TYPE datetime VALUE $before OR time::now();