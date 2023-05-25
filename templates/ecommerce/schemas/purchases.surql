# in: customer
# out: product
DEFINE TABLE purchases SCHEMALESS
    PERMISSIONS
        FOR select WHERE in = $auth.id
        FOR create, update, delete NONE;

DEFINE FIELD quantity ON purchases TYPE number;
DEFINE FIELD shipping_address ON purchases TYPE option<record<address>>;
DEFINE FIELD created_at ON purchases TYPE datetime DEFAULT time::now();
DEFINE FIELD shipped_at ON purchases TYPE option<datetime>;
DEFINE FIELD total ON purchases TYPE number;
DEFINE FIELD status ON purchases TYPE string DEFAULT 'Pending' ASSERT $value IN ['Pending', 'Delivered'];