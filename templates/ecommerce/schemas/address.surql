DEFINE TABLE OVERWRITE address SCHEMALESS
    PERMISSIONS
        FOR select, create, update, delete WHERE customer = $auth.id;

DEFINE FIELD OVERWRITE number ON address TYPE string;
DEFINE FIELD OVERWRITE street ON address TYPE string;
DEFINE FIELD OVERWRITE city ON address TYPE string;
DEFINE FIELD OVERWRITE state ON address TYPE option<string>;
DEFINE FIELD OVERWRITE zip_code ON address TYPE option<string>;
DEFINE FIELD OVERWRITE country ON address TYPE string;
DEFINE FIELD OVERWRITE customer ON address TYPE record<customer>;