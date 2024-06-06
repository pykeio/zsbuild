#ifndef _ZSB_COMMON
#define _ZSB_COMMON

#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>

struct Location {
	const char *file;
	size_t file_len;
	const char *namespace_;
	size_t namespace_len;
	int64_t line;
	int64_t column;
	int64_t length;
	const char *line_text;
	size_t line_text_len;
	const char *suggestion;
	size_t suggestion_len;
};

struct Note {
	const char *text;
	size_t text_len;
	struct Location *location;
};

struct Message {
	const char *id;
	size_t id_len;
	const char *plugin_name;
	size_t plugin_name_len;
	const char *text;
	size_t text_len;
	struct Location *location;
	struct Note *notes;
	size_t notes_len;
};

struct ContextResult {
	struct Message *messages;
	size_t messages_len;
};

struct OutputFile {
	const char *path;
	size_t path_len;
	const char *hash;
	size_t hash_len;
	const char *contents;
	size_t contents_len;
};

struct BuildResult {
	struct OutputFile *output_files;
	size_t output_files_len;
	struct Message *errors;
	size_t errors_len;
	struct Message *warnings;
	size_t warnings_len;
};

typedef void (*BuildAsyncCallback)(struct BuildResult *result, void *data);
void Zsb_BuildAsyncCallback_Dispatch(BuildAsyncCallback callback, struct BuildResult *result, void *data);

typedef void (*PluginBuildCallback)(uint64_t handle, void *data);
void Zsb_PluginBuildCallback_Dispatch(PluginBuildCallback callback, uint64_t handle, void *data);
typedef void (*PluginDestructor)(void *data);
void Zsb_PluginDestructor_Dispatch(PluginDestructor callback, void *data);

struct PluginOnStartResult {
	struct Message *errors;
	size_t errors_len;
	struct Message *warnings;
	size_t warnings_len;
};
void Zsb_PluginOnStartResult_Destroy(struct PluginOnStartResult *res);

typedef struct PluginOnStartResult *(*PluginCallbackOnStart)(void *data);
struct PluginOnStartResult *Zsb_PluginCallbackOnStart_Dispatch(PluginCallbackOnStart callback, void *data);

#endif
