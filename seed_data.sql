-- Seed data for e2e tests
-- Inserts minimal test data for common test scenarios

-- Data group: 2024 project (for tests)
INSERT INTO data_groups (id, name, type, created_at, bills_storage_path) 
VALUES (1, '2024', 'project', '2026-01-01 00:00:00', 'pdf_imgs/2024')
ON CONFLICT (name) DO NOTHING;

-- Data group: 2025 organization
INSERT INTO data_groups (id, name, type, created_at, bills_storage_path) 
VALUES (2, '2025', 'organization', '2026-01-01 00:00:00', 'pdf_imgs/2025')
ON CONFLICT (name) DO NOTHING;

-- Sample bills
INSERT INTO bills (data_group, filename, amount, date, is_cash) VALUES
(1, 'test_invoice_1.pdf', 100.00, '2024-01-15', false),
(1, 'test_cash_receipt.pdf', 25.50, '2024-01-20', true),
(2, 'test_invoice_2.pdf', 200.00, '2025-02-01', false)
ON CONFLICT DO NOTHING;

-- Sample application reports
INSERT INTO application_reports (name, amount, created_at, submission_deadline, data_group) VALUES
('Annual Report 2024', 1500.00, '2024-12-31', '2025-03-31', 1),
('Q1 Report 2025', 500.00, '2025-03-31', '2025-06-30', 2)
ON CONFLICT DO NOTHING;

-- Sample expenses
INSERT INTO expenses (data_group, date, partner, amount, expense_type, bill, application, is_cash) VALUES
(1, '2024-01-15', 'Test Partner 1', 100.00, 1, 1, NULL, false),
(1, '2024-01-20', 'Cash Expense', 25.50, 2, 2, NULL, true),
(2, '2025-02-01', 'Test Partner 2', 200.00, 1, 3, 2, false)
ON CONFLICT DO NOTHING;