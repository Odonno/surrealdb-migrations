DEFINE TABLE OVERWRITE product SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE name ON product TYPE string;
DEFINE FIELD OVERWRITE description ON product TYPE string;
DEFINE FIELD OVERWRITE price ON product TYPE number;
DEFINE FIELD OVERWRITE category ON product TYPE string;
DEFINE FIELD OVERWRITE images ON product TYPE array<string>;