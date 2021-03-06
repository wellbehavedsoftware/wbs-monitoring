#include <apt-pkg/cachefile.h>
#include <apt-pkg/pkgcache.h>

#include <inttypes.h>
#include <signal.h>

// ========== public interface

struct AptcUpgradeSummary {
	uint64_t upgrade;
	uint64_t remove;
	uint64_t install;
	uint64_t broken;
	uint64_t bad;
	uint64_t reserved05;
	uint64_t reserved06;
	uint64_t reserved07;
	uint64_t reserved08;
	uint64_t reserved09;
	uint64_t reserved10;
	uint64_t reserved11;
	uint64_t reserved12;
	uint64_t reserved13;
	uint64_t reserved14;
	uint64_t reserved15;
};

extern "C" {

	bool aptc_init ();

	bool aptc_upgrade_summary_get (
		AptcUpgradeSummary * summary);

	void aptc_configuration_set_string (
		const char * name,
		const char * value);

	const char * aptc_error_message ();

}

// ========== internal stuff

class AptCacheState {

public:

	bool initialized;
	bool failed;

	bool debug;
	string error_message;

	AptCacheState () :
		initialized (false),
		failed (false),
		debug (false) {

	}

};

#define debug(format, ...) \
	if (state.debug) { fprintf (stderr, format, ## __VA_ARGS__); }

static AptCacheState state;

// ========== implementation

void aptc_configuration_set_string (
		const char * name,
		const char * value) {

	_config->Set (
		name,
		value);

}

const char * aptc_error_message () {

	return state.error_message.c_str ();

}

bool aptc_init () {

	debug (
		"Aptc initialize\n");

	if (state.initialized) {
		return true;
	}

	if (state.failed) {
		return false;
	}

	// init config

	debug (
		"  Init config\n");

	{

		bool init_config_result =
			pkgInitConfig (
				* _config);

		if (! init_config_result) {

			state.error_message =
				"Call to pkgInitConfig failed";

			return false;

		}

	}

	// init system

	debug (
		"  Init system\n");

	{

		bool init_system_result =
			pkgInitSystem (
				* _config,
				_system);

		if (! init_system_result) {

			state.error_message =
				"Call to pkgInitSystem failed";

			goto error;

		}

	}

	state.initialized = true;

	return true;

error:

	debug (
		"  Error\n");

	state.failed = true;

	return false;

}

bool aptc_upgrade_summary_get (
		AptcUpgradeSummary * summary) {

	if (! aptc_init ()) {
		return false;
	}

	debug (
		"Aptc upgrade summary get\n");

	// simulate the upgrade

	debug (
		"  Open cache files\n");

	pkgCacheFile cache_file;

	if (! cache_file.Open ()) {

		state.error_message =
			"Call to pkgCacheFile.Open failed";

		return false;

	}

	debug (
		"  Build caches\n");

	if (! cache_file.BuildCaches ()) {

		state.error_message =
			"Call to pkgCacheFile.BuildCaches failed";

		return false;

	}

	debug (
		"  Build dep cache\n");

	if (! cache_file.BuildDepCache ()) {

		state.error_message =
			"Call to pkgCacheFile.BuildDepCache failed";

		return false;

	}

	pkgDepCache * dep_cache =
		cache_file.GetDepCache ();

	// update struct

	summary->upgrade =
		dep_cache->KeepCount ();

	summary->remove =
		dep_cache->DelCount ();

	summary->install =
		dep_cache->InstCount ();

	summary->broken =
		dep_cache->BrokenCount ();

	summary->bad =
		dep_cache->BadCount ();

	// return
	
	return true;

	/*
	debug (
		"  Find upgraded packages\n");

	for (
		pkgCache::PkgIterator package =
			dep_cache->PkgBegin ();
		! package.end ();
		package ++
	) {

		const char * current_version =
			package.CurVersion ();

		if (! current_version) {
			continue;
		}

		const char * candidate_version =
			package.CandVersion ();

		if (! strcmp (
				current_version,
				candidate_version)) {
			continue;
		}

		std::cout
			<< "    - "
			<< package.Name ()
			<< " "
			<< current_version
			<< " -> "
			<< candidate_version
			<< std::endl;

	}
	*/

}

// ========== just for testing

/*
int main () {

	state.debug = true;

	AptcUpgradeSummary summary;

	aptc_upgrade_summary_get (
		& summary);

	debug (
		"Upgrade summary\n");

	debug (
		"  Upgrade %ld\n",
		summary.upgrade);

	debug (
		"  Delete %ld\n",
		summary.remove);

	debug (
		"  Install %ld\n",
		summary.install);

	debug (
		"  Broken %ld\n",
		summary.broken);

	debug (
		"  Bad %ld\n",
		summary.bad);

	return EXIT_SUCCESS;

}
*/

// ex: noet ts=4 filetype=cc1
