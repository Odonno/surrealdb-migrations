DEFINE TABLE OVERWRITE purchases SCHEMALESS
    TYPE RELATION IN customer OUT product
    PERMISSIONS
        FOR select WHERE in = $auth.id
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE quantity ON purchases TYPE number;
DEFINE FIELD OVERWRITE shipping_address ON purchases TYPE option<record<address>>;
DEFINE FIELD OVERWRITE created_at ON purchases TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD OVERWRITE shipped_at ON purchases TYPE option<datetime>;
DEFINE FIELD OVERWRITE total ON purchases TYPE number;
DEFINE FIELD OVERWRITE status ON purchases TYPE string DEFAULT 'Pending' ASSERT $value IN ['Pending', 'Delivered'];