class InventoryError(Exception):
    """Domain error with a stable machine-readable code."""

    def __init__(self, code, message, **context):
        super().__init__(message)
        self.code = str(code).upper()
        self.message = str(message)
        self.context = dict(context)

    def __str__(self):
        return f"{self.code}: {self.message}"


def fail(code, message, **context):
    raise InventoryError(code, message, **context)
