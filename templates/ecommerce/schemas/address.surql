DEFINE TABLE address SCHEMALESS
    PERMISSIONS
        FOR select, create, update, delete WHERE customer = $auth.id;

DEFINE FIELD number ON address TYPE string;
DEFINE FIELD street ON address TYPE string;
DEFINE FIELD city ON address TYPE string;
DEFINE FIELD state ON address TYPE string;
DEFINE FIELD zip_code ON address TYPE string;
DEFINE FIELD country ON address TYPE string;
DEFINE FIELD customer ON address TYPE record(customer);