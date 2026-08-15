CREATE TABLE `agent_profiles` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`provider` text NOT NULL,
	`model` text,
	`instructions` text,
	`archived_at` integer,
	`created_at` integer NOT NULL,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `squad_members` (
	`squad_id` text NOT NULL,
	`member_kind` text NOT NULL,
	`member_id` text NOT NULL,
	`role_note` text,
	`position` integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX `squad_members_by_squad` ON `squad_members` (`squad_id`,`position`);--> statement-breakpoint
CREATE TABLE `squads` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`leader_profile_id` text NOT NULL,
	`instructions` text,
	`archived_at` integer,
	`created_at` integer NOT NULL,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `work_item_comments` (
	`id` text PRIMARY KEY NOT NULL,
	`work_item_id` text NOT NULL,
	`author_kind` text NOT NULL,
	`author_id` text,
	`content` text NOT NULL,
	`created_at` integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX `work_item_comments_by_item` ON `work_item_comments` (`work_item_id`,`created_at`);--> statement-breakpoint
CREATE TABLE `work_item_details` (
	`work_item_id` text PRIMARY KEY NOT NULL,
	`data` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `work_items` (
	`id` text PRIMARY KEY NOT NULL,
	`key_number` integer NOT NULL,
	`title` text NOT NULL,
	`workflow_status` text NOT NULL,
	`important` integer DEFAULT true NOT NULL,
	`urgent` integer DEFAULT false NOT NULL,
	`flagged` integer DEFAULT false NOT NULL,
	`assignee_kind` text,
	`assignee_id` text,
	`project_id` text,
	`created_at` integer NOT NULL,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX `work_items_by_updated_at` ON `work_items` (`updated_at`);--> statement-breakpoint
ALTER TABLE `sessions` ADD `workflow_status` text DEFAULT 'todo' NOT NULL;--> statement-breakpoint
ALTER TABLE `sessions` ADD `important` integer DEFAULT true NOT NULL;--> statement-breakpoint
ALTER TABLE `sessions` ADD `urgent` integer DEFAULT false NOT NULL;--> statement-breakpoint
ALTER TABLE `sessions` ADD `flagged` integer DEFAULT false NOT NULL;--> statement-breakpoint
ALTER TABLE `sessions` ADD `work_item_id` text;--> statement-breakpoint
ALTER TABLE `sessions` ADD `agent_profile_id` text;