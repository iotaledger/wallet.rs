import iota_wallet as iw


def test_event_balance_change():
    def on_balance_changed(event):
        assert isinstance(event, str)
    event_id = iw.on_balance_change(on_balance_changed)
    iw.remove_balance_change_listener(bytes(event_id))


def test_event_new_transaction():
    def on_new_transaction(event):
        assert isinstance(event, str)
    event_id = iw.on_new_transaction(on_new_transaction)
    iw.remove_new_transaction_listener(bytes(event_id))


def test_event_confirmation_state_change():
    def on_confirmation_state_change(event):
        assert isinstance(event, str)
    event_id = iw.on_confirmation_state_change(on_confirmation_state_change)
    iw.remove_confirmation_state_change_listener(bytes(event_id))


def test_event_reattachment():
    def on_reattachment(event):
        assert isinstance(event, str)
    event_id = iw.on_reattachment(on_reattachment)
    iw.remove_reattachment_listener(bytes(event_id))


def test_event_broadcast():
    def on_broadcast(event):
        assert isinstance(event, str)
    event_id = iw.on_broadcast(on_broadcast)
    iw.remove_broadcast_listener(bytes(event_id))


def test_event_error():
    def on_error(event):
        assert isinstance(event, str)
    event_id = iw.on_error(on_error)
    iw.remove_error_listener(bytes(event_id))


def test_event_stronghold_status_change():
    def on_stronghold_status_change(event):
        assert isinstance(event, str)
    event_id = iw.on_stronghold_status_change(on_stronghold_status_change)
    iw.remove_stronghold_status_change_listener(bytes(event_id))
