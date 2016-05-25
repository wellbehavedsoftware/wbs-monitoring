#include <apt-pkg/cachefile.h>
#include <apt-pkg/pkgcache.h>

#include <signal.h>

class AptCacheState {

public:

	bool initialized;
	bool failed;

	AptCacheState () :
		initialized (false),
		failed (false) {

	}

};

static AptCacheState state;

extern "C" {

	bool aptc_init ();

}

bool aptc_init () {

	if (state.initialized) {
		return true;
	}

	if (state.failed) {
		return false;
	}

	// init config

	printf (
		"  Init config\n");

	{

		bool init_config_result =
			pkgInitConfig (
				* _config);

		if (! init_config_result) {
			return false;
		}

	}

	// init system

	printf (
		"  Init system\n");

	{

		bool init_system_result =
			pkgInitSystem (
				* _config,
				_system);

		if (! init_system_result) {
			goto error;
		}

	}

	state.initialized = true;

	return true;

error:

	printf (
		"  Error\n");

	state.failed = true;

	return false;

}

void do_some_stuff () {

	pkgCacheFile cache_file;

	// simulate the upgrade

	printf (
		"  Open cache files\n");

	if (! cache_file.Open ()) {
		return;
	}

	printf (
		"  Build caches\n");

	if (! cache_file.BuildCaches ()) {
		return;
	}

	printf (
		"  Build dep cache\n");

	if (! cache_file.BuildDepCache ()) {
		return;
	}

	printf (
		"  Show upgrade details\n");

	pkgDepCache * dep_cache =
		cache_file.GetDepCache ();

	printf (
		"    Upgrade %ld\n",
		dep_cache->KeepCount ());

	printf (
		"    Delete %ld\n",
		dep_cache->DelCount ());

	printf (
		"    Install %ld\n",
		dep_cache->InstCount ());

	printf (
		"    Broken %ld\n",
		dep_cache->BrokenCount ());

	printf (
		"    Bad %ld\n",
		dep_cache->BadCount ());

	return;

	printf (
		"  Find upgraded packages\n");

	pkgCache * cache =
		cache_file.GetPkgCache ();

	for (
		pkgCache::PkgIterator package = cache->PkgBegin ();
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

}

int main () {

	printf (
		"About to init\n");

	if (! aptc_init ()) {

		printf (
			"Init failed\n");

	}

	printf (
		"About to do some stuff\n");

	do_some_stuff ();

}

// ex: noet ts=4 filetype=cc1
