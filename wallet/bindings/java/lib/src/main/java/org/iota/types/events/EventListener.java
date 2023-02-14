package org.iota.types.events;

public interface EventListener {
    /**
     * Event callback triggered at certain wallet actions
     * 
     * @param event The event
     */
    public void receive(Event event);
}
