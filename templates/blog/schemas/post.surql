DEFINE TABLE post SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_post IN $auth.permissions
        FOR update, delete WHERE author = $auth.id;

DEFINE FIELD title ON post TYPE string;
DEFINE FIELD content ON post TYPE string;
DEFINE FIELD author ON post TYPE record<user>;
DEFINE FIELD created_at ON post TYPE datetime DEFAULT time::now();
DEFINE FIELD status ON post TYPE string DEFAULT 'DRAFT' ASSERT $value IN ['DRAFT', 'PUBLISHED'];