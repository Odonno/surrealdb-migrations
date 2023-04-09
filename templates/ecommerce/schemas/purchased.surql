# in: customer
# out: product
DEFINE TABLE purchased SCHEMALESS;

DEFINE FIELD quantity ON purchased TYPE number;
DEFINE FIELD shipping_address ON purchased TYPE record(address);
DEFINE FIELD created_at ON purchased TYPE datetime VALUE $before OR time::now();
DEFINE FIELD shipped_at ON purchased TYPE datetime;
DEFINE FIELD total ON purchased TYPE number;
DEFINE FIELD status ON purchased TYPE string VALUE $value OR $before OR 'Pending' ASSERT $value == NONE OR $value INSIDE ['Pending', 'Delivered'];