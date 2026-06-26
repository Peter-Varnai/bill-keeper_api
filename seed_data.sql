-- Seed data for tests

INSERT INTO data_groups (id, name, type, created_at, bills_storage_path, user_id)
VALUES (1, '2024', 'project', '2024-01-01 00:00:00', 'pdf_imgs/2024', 1);

SELECT setval('data_groups_id_seq', GREATEST((SELECT COALESCE(MAX(id), 0) FROM data_groups), 1));

INSERT INTO bills (id, data_group, filename, amount, date, is_cash) VALUES
(1, 1, 'test_invoice_1.pdf', 100.00, '2024-01-15', false),
(2, 1, 'test_receipt_2.jpg',  50.00, '2024-02-03', false);

SELECT setval('bills_id_seq', GREATEST((SELECT COALESCE(MAX(id), 0) FROM bills), 1));

INSERT INTO application_reports (id, name, amount, data_group) VALUES
(1, 'Test Report 2024', 500.00, 1);

SELECT setval('application_reports_id_seq', GREATEST((SELECT COALESCE(MAX(id), 0) FROM application_reports), 1));

INSERT INTO expenses (data_group, date, partner, amount, expense_type, bill, application, is_cash) VALUES
(1, '2024-01-15', 'Test Partner A', 100.00, 1, 1, 1, false),
(1, '2024-01-20', 'Test Partner B',  50.00, 6, 2, 1, true);

INSERT INTO utility_data (data_group, key, value) VALUES
(1, 'bank_stand', 1000.00),
(1, 'cash_stand',  100.00);
