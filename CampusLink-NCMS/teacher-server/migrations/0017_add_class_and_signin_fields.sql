-- 为班级、签到密码与座位号绑定补充字段

-- students: 所属班级、密码是否已由学生自定义
ALTER TABLE students ADD COLUMN class_id TEXT REFERENCES classes(id) ON DELETE SET NULL;
ALTER TABLE students ADD COLUMN password_set INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_students_class_id ON students(class_id);

-- student_devices: 所属班级、固定座位号、待补发签到标志
ALTER TABLE student_devices ADD COLUMN class_id TEXT REFERENCES classes(id) ON DELETE SET NULL;
ALTER TABLE student_devices ADD COLUMN seat_no TEXT;
ALTER TABLE student_devices ADD COLUMN pending_checkin INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_student_devices_class_id ON student_devices(class_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_student_devices_class_seat ON student_devices(class_id, seat_no) WHERE seat_no IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_students_class_seat ON students(class_id, seat_no) WHERE seat_no IS NOT NULL;

-- attendance_records: 签到时的座位号快照
ALTER TABLE attendance_records ADD COLUMN seat_no TEXT;
