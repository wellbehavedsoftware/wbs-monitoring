#[ macro_export ]
macro_rules! check {

	(

		new = $ new : ident ,
		name = $ name : expr ,
		prefix = $ prefix : expr ,

		provider = $ provider : ident ,

		instance = $ instance : ident {
			$ ( $ instance_definition : tt ) *
		},

		options_spec = | $ options_spec : ident | {
			$ ( $ options_spec_definition : tt ) *
		},

		options_parse = | $ options_matches : ident | {
			$ ( $ options_parse_definition : tt ) *
		},

	) => {

		pub fn $ new (
		) -> Box <PluginProvider> {

			Box::new (
				$ provider {},
			)

		}

		struct $ provider {
		}

		struct $ instance {
			$ ( $ instance_definition ) *
		}

		impl PluginProvider
		for $ provider {

			fn name (& self) -> & str {
				$ name
			}

			fn prefix (& self) -> & str {
				$ prefix
			}

			fn build_options_spec (
				& self,
			) -> getopts::Options {

				let mut $ options_spec =
					getopts::Options::new ();

				$ options_spec.optflag (
					"",
					"help",
					"print this help menu");

				$ ( $ options_spec_definition ) *

				$ options_spec

			}

			fn new_instance (
				& self,
				_options_spec: & getopts::Options,
				$ options_matches: & getopts::Matches,
			) -> Result <Box <PluginInstance>, Box <error::Error>> {

				Ok (Box::new ({
					$ ( $ options_parse_definition ) *
				}))

			}

		}

	}

}

// ex: noet ts=4 filetype=rust
