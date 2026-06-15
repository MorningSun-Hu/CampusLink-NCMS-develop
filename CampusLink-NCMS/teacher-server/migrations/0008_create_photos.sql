-- 创建 photos 表
-- 用途：保存告警和检查相关的图片上传记录

CREATE TABLE photos (
    id TEXT PRIMARY KEY,
    alert_id INTEGER,
    inspection_id TEXT,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL DEFAULT 0,
    mime_type TEXT NOT NULL DEFAULT 'image/jpeg',
    upload_time TEXT NOT NULL DEFAULT (datetime('now')),
    description TEXT,
    FOREIGN KEY (inspection_id) REFERENCES inspection_records(id) ON DELETE SET NULL
);

CREATE INDEX idx_photos_alert_id ON photos(alert_id);
CREATE INDEX idx_photos_inspection_id ON photos(inspection_id);
CREATE INDEX idx_photos_upload_time ON photos(upload_time);
