-- First improvement script: Test the GPU pipeline with a simple program

INSERT OR REPLACE INTO improvement_scripts (name, lang, purpose, code, enabled, created_at)
VALUES (
    'hello_gpu',
    'pixel',
    'Test GPU pipeline with simple text rendering',
    'TXT 10 10 HELLO GPU
HALT',
    1,
    datetime('now')
);

-- Add a development goal
INSERT INTO development_goals (goal, priority, created_at)
VALUES ('Validate end-to-end GPU execution pipeline', 10, datetime('now'));
