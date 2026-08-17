CREATE TABLE `library_assets` (
	`id` text PRIMARY KEY NOT NULL,
	`filename` text NOT NULL,
	`prompt` text,
	`source_path` text,
	`session_id` text,
	`provider` text,
	`model` text,
	`created_at` integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX `library_assets_by_created_at` ON `library_assets` (`created_at`);