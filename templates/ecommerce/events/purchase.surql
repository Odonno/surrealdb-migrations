DEFINE TABLE OVERWRITE purchase SCHEMALESS
    PERMISSIONS
        FOR select, create FULL
        FOR update, delete NONE;

DEFINE FIELD OVERWRITE customer ON purchase TYPE record<customer>;
DEFINE FIELD OVERWRITE product ON purchase TYPE record<product>;
DEFINE FIELD OVERWRITE quantity ON purchase TYPE number;
DEFINE FIELD OVERWRITE total ON purchase TYPE number;

DEFINE EVENT OVERWRITE purchase ON TABLE purchase WHEN $event == "CREATE" THEN {
    LET $from = (SELECT * FROM customer WHERE id == $after.customer);
    LET $to = (SELECT * FROM product WHERE id == $after.product);

    RELATE $from->purchases->$to CONTENT {
        quantity: $after.quantity,
        total: $after.total,
        status: 'Pending',
    };
};