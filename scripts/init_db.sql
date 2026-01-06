-- AgentMem Database Initialization Script
-- This script initializes the database with default data

-- Create default organization
INSERT OR IGNORE INTO organizations (id, name, created_at, updated_at)
VALUES ('default-org', 'Default Organization', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- Create default user
INSERT OR IGNORE INTO users (id, organization_id, email, name, created_at, updated_at)
VALUES (
    'user-' || lower(hex(randomblob(16))),
    'default-org',
    'admin@agentmem.dev',
    'Admin User',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Create sample agents
INSERT OR IGNORE INTO agents (id, organization_id, name, description, state, created_at, updated_at)
VALUES 
    ('agent-' || lower(hex(randomblob(16))), 'default-org', 'Customer Support Bot', '24/7 customer support agent', 'idle', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('agent-' || lower(hex(randomblob(16))), 'default-org', 'Research Assistant', 'Helps with research tasks', 'idle', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('agent-' || lower(hex(randomblob(16))), 'default-org', 'Code Reviewer', 'Reviews code and provides feedback', 'idle', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- Print success message
SELECT 'Database initialized successfully' as message;

