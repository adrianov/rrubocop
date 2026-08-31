Regexp.last_match(1)
Regexp.last_match(9)
Regexp.last_match(0)
Regexp.last_match.pre_match
Regexp.last_match.post_match
Regexp.last_match(-1)

# $0 / $00 are program name, not regexp backrefs
_name = $0
_also = $00
