filter :from_channels, as: :select,
                       collection: enum_to_options.call('channels', use_raw_values: true),
                       multiple: true,
                       label: 'channels'
