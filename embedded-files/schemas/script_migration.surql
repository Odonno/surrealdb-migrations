DEFINE TABLE OVERWRITE script_migration SCHEMAFULL;

DEFINE FIELD OVERWRITE script_name ON script_migration TYPE string;
DEFINE FIELD OVERWRITE executed_at ON script_migration TYPE datetime DEFAULT time::now();