"""Select the observed native thread without weakening target assertions."""


def binding_for_thread(bindings, thread_id):
    return next(binding for binding in bindings if binding["document"].get("thread_id") == thread_id)
