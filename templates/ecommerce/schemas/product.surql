DEFINE TABLE product SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD name ON product TYPE string;
DEFINE FIELD description ON product TYPE string;
DEFINE FIELD price ON product TYPE number;
DEFINE FIELD category ON product TYPE string;
DEFINE FIELD images ON product TYPE array;
DEFINE FIELD images.* ON product TYPE string;