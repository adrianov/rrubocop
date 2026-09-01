# %Q with quotes + escapes — RuboCop keeps percent form
XML = %Q(<?xml version="1.0" encoding="UTF-8"?>\n)
html = %Q(<tag attr="#{val}"></tag>)
both = %q(it's a "quote")
