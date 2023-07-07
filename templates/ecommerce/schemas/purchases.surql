# in: customer
# out: product
DEFINE TABLE purchases SCHEMALESS
    PERMISSIONS
        FOR select WHERE in = $auth.id
        FOR create, update, delete NONE;

DEFINE FIELD quantity ON purchases TYPE number;
DEFINE FIELD shipping_address ON purchases TYPE record(address);
DEFINE FIELD created_at ON purchases TYPE datetime VALUE $before OR time::now();
DEFINE FIELD shipped_at ON purchases TYPE datetime;
DEFINE FIELD total ON purchases TYPE number;
DEFINE FIELD status ON purchases TYPE string VALUE $value OR $before OR 'Pending' ASSERT $value == NONE OR $value INSIDE ['Pending', 'Delivered'];