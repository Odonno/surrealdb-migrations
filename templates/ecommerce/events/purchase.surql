DEFINE TABLE purchase SCHEMALESS
    PERMISSIONS
        FOR select, create FULL
        FOR update, delete NONE;

DEFINE FIELD customer ON purchase TYPE record<customer>;
DEFINE FIELD product ON purchase TYPE record<product>;
DEFINE FIELD quantity ON purchase TYPE number;
DEFINE FIELD total ON purchase TYPE number;

DEFINE EVENT purchase ON TABLE purchase WHEN $event == "CREATE" THEN {
    LET $from = (SELECT * FROM customer WHERE id == $after.customer);
    LET $to = (SELECT * FROM product WHERE id == $after.product);

    RELATE $from->purchases->$to CONTENT {
        quantity: $after.quantity,
        total: $after.total,
        status: 'Pending',
    };
};