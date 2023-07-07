RELATE customer:pratim->purchases->product:iphone CONTENT {
    quantity: 1,
    total: 600,
    status: 'Pending',
};

RELATE customer:pratim->purchases->product:shirt CONTENT {
    quantity: 2,
    total: 40,
    status: 'Delivered',
};

RELATE customer:alex->purchases->product:iphone CONTENT {
    quantity: 1,
    total: 600,
    status: 'Pending',
};

RELATE customer:alex->purchases->product:shirt CONTENT {
    quantity: 2,
    total: 12,
    status: 'Delivered',
};

RELATE customer:tobie->purchases->product:iphone CONTENT {
    quantity: 1,
    total: 600,
    status: 'Pending',
};