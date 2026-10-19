`join_orders.py CUSTOMERS ORDERS` joins two CSV files and writes CSV to stdout. `CUSTOMERS` has columns `customer_id,name`. `ORDERS` has columns `order_id,customer_id,total_cents,notes`.

The output header must be `order_id,customer_id,name,total_cents,notes`. Emit one output row for every order, in the same order as the orders file. Customer names and notes may contain quoted commas, and duplicate customer IDs in the orders file must not be collapsed.

Keep the script dependency-free.
