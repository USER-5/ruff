with aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:
    ...
    # trailing

with a, a:  # after colon
    ...
    # trailing

with (
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
):
    ...
    # trailing


with (
        a  # a
        ,  # comma
        b  # c
        ): # colon
    ...


with (
        a  # a
        as  # as
        b  # b
        ,  # comma
        c  # c
        ): # colon
    ...  # body
    # body trailing own


with (a,):  # magic trailing comma
    ...


with (a):  # should remove brackets
    ...

# TODO: black doesn't wrap this, but maybe we want to anyway?
# if we do want to wrap, do we prefer to wrap the entire WithItem or to let the
# WithItem allow the `aa + bb` content expression to be wrapped
with aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb as c:
    ...


# currently unparsable by black: https://github.com/psf/black/issues/3678
with (name_2 for name_0 in name_4):
    pass
with (a, *b):
    pass
