DEFINE TABLE permission SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD name ON permission TYPE string;
DEFINE FIELD created_at ON permission TYPE datetime VALUE time::now() READONLY;

DEFINE INDEX unique_name ON permission COLUMNS name UNIQUE;