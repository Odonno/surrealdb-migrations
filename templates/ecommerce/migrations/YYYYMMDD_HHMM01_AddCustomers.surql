INSERT INTO address {
    id: 'pratim_home', 
    number: '221B', 
    street: 'Baker street', 
    city: 'London', 
    country: 'UK',
    customer: customer:pratim
};

INSERT INTO customer {
    id: 'pratim',
    name: 'Pratim',
    email: 'abc@gmail.com',
    password: '12345678',
    addresses: [address:pratim_home]
};

INSERT INTO address {
    id: 'tobie_home', 
    number: '221A', 
    street: 'Church street', 
    city: 'London', 
    country: 'UK',
    customer: customer:tobie
};

INSERT INTO customer {
    id: 'tobie',
    name: 'Tobie',
    email: 'tobie@gmail.com',
    password: '12345678',
    addresses: [address:tobie_home]
};

INSERT INTO address {
    id: 'alex_home', 
    number: '221C', 
    street: 'Pound street', 
    city: 'London', 
    country: 'UK',
    customer: customer:alex
};

INSERT INTO customer {
    id: 'alex',
    name: 'Alex',
    email: 'alex@gmail.com',
    password: '12345678',
    addresses: [address:alex_home]
};