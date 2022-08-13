DROP TRIGGER IF EXISTS set_updated_at ON institutions;
SELECT diesel_manage_updated_at('institutions');
