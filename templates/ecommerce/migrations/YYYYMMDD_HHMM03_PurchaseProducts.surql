RELATE customer:pratim->purchased->product:iphone CONTENT {
    quantity: 1,
    total: 600,
    status: 'Pending',
};

RELATE customer:pratim->purchased->product:shirt CONTENT {
    quantity: 2,
    total: 40,
    status: 'Delivered',
};

RELATE customer:alex->purchased->product:iphone CONTENT {
    quantity: 1,
    total: 600,
    status: 'Pending',
};

RELATE customer:alex->purchased->product:shirt CONTENT {
    quantity: 2,
    total: 12,
    status: 'Delivered',
};

RELATE customer:tobie->purchased->product:iphone CONTENT {
    quantity: 1,
    total: 600,
    status: 'Pending',
};