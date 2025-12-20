-- Sample Data for Kipko POS
-- This migration inserts sample data for testing and demonstration

-- Sample tables
INSERT INTO tables (number, capacity, status, location) VALUES
(1, 4, 'Empty', 'Main Floor'),
(2, 4, 'Empty', 'Main Floor'),
(3, 2, 'Empty', 'Bar Area'),
(4, 6, 'Empty', 'Patio'),
(5, 8, 'Empty', 'Private Room'),
(6, 4, 'Empty', 'Main Floor'),
(7, 2, 'Empty', 'Bar Area'),
(8, 4, 'Empty', 'Patio');

-- Sample menu items
INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Caesar Salad',
    'Fresh romaine lettuce with parmesan cheese and croutons',
    8.50,
    8.50,
    10,
    1
FROM menu_item_categories c WHERE c.name = 'Appetizers';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Soup of the Day',
    'Daily chef''s special soup',
    6.25,
    8.50,
    5,
    2
FROM menu_item_categories c WHERE c.name = 'Appetizers';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Garlic Bread',
    'Toasted bread with garlic butter and herbs',
    5.75,
    8.50,
    8,
    3
FROM menu_item_categories c WHERE c.name = 'Appetizers';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Classic Burger',
    '8oz beef patty with lettuce, tomato, onion, and pickles',
    12.95,
    8.50,
    15,
    1
FROM menu_item_categories c WHERE c.name = 'Main Courses';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Grilled Chicken Sandwich',
    'Grilled chicken breast with avocado and bacon',
    14.50,
    8.50,
    18,
    2
FROM menu_item_categories c WHERE c.name = 'Main Courses';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Fish and Chips',
    'Beer-battered cod with golden fries',
    16.75,
    8.50,
    20,
    3
FROM menu_item_categories c WHERE c.name = 'Main Courses';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Vegetarian Pasta',
    'Pasta with seasonal vegetables in garlic sauce',
    13.25,
    8.50,
  12,
    4
FROM menu_item_categories c WHERE c.name = 'Main Courses';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Coffee',
    'Freshly brewed coffee',
    3.25,
    8.50,
    3,
    1
FROM menu_item_categories c WHERE c.name = 'Beverages';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Iced Tea',
    'Fresh brewed iced tea',
    2.95,
    8.50,
    3,
    2
FROM menu_item_categories c WHERE c.name = 'Beverages';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Soda',
    'Coca-Cola, Pepsi, or Sprite',
    2.75,
    8.50,
    2,
    3
FROM menu_item_categories c WHERE c.name = 'Beverages';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Chocolate Cake',
    'Rich chocolate cake with ganache',
    7.50,
    8.50,
    5,
    1
FROM menu_item_categories c WHERE c.name = 'Desserts';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Ice Cream',
    'Vanilla, chocolate, or strawberry',
    4.25,
    8.50,
    2,
    2
FROM menu_item_categories c WHERE c.name = 'Desserts';

INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order) 
SELECT 
    c.id,
    'Fruit Tart',
    'Fresh seasonal fruit on pastry cream',
    6.75,
    8.50,
    8,
    3
FROM menu_item_categories c WHERE c.name = 'Desserts';

-- Sample staff members
INSERT INTO staff (name, email, role) VALUES
('John Smith', 'john@kipko.com', 'Server'),
('Jane Doe', 'jane@kipko.com', 'Server'),
('Mike Johnson', 'mike@kipko.com', 'Manager'),
('Sarah Wilson', 'sarah@kipko.com', 'Kitchen'),
('Tom Brown', 'tom@kipko.com', 'Host');
