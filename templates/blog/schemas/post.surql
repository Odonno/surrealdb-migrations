DEFINE TABLE OVERWRITE post SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_post IN $auth.permissions
        FOR update, delete WHERE author = $auth.id;

DEFINE FIELD OVERWRITE title ON post TYPE string;
DEFINE FIELD OVERWRITE content ON post TYPE string;
DEFINE FIELD OVERWRITE author ON post TYPE record<user>;
DEFINE FIELD OVERWRITE created_at ON post TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD OVERWRITE status ON post TYPE string DEFAULT 'DRAFT' ASSERT $value IN ['DRAFT', 'PUBLISHED'];