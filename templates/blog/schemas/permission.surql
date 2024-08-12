DEFINE TABLE OVERWRITE permission SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE name ON permission TYPE string;
DEFINE FIELD OVERWRITE created_at ON permission TYPE datetime VALUE time::now() READONLY;

DEFINE INDEX OVERWRITE unique_name ON permission COLUMNS name UNIQUE;