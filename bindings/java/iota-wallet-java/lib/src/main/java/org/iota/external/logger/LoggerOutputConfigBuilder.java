package org.iota.external.logger;

public class LoggerOutputConfigBuilder {
    private String name;
    private LevelFilter levelFilter;
    private String[] targetFilters;
    private String[] targetExclusions;
    private boolean colorEnabled;

    /**
     * Set the name of an output file, or `stdout` for standard output.
     * 
     * @param name Name of an output file
     * @return the builder
     */
    public LoggerOutputConfigBuilder setName(String name) {
        this.name = name;
        return this;
    }

    /**
     * Set log level filter of an output.
     * 
     * @param levelFilter level filter of an output.
     * @return the builder
     */
    public LoggerOutputConfigBuilder setLevelFilter(LevelFilter levelFilter) {
        this.levelFilter = levelFilter;
        return this;
    }

    /**
     * Set log target filters of an output.
     * 
     * @param targetFilters target filters of an output.
     * @return the builder
     */
    public LoggerOutputConfigBuilder setTargetFilters(String[] targetFilters) {
        this.targetFilters = targetFilters;
        return this;
    }

    /**
     * Set log target exclusions of an output.
     * 
     * @param targetExclusions Log target exclusions of an output.
     * @return the builder
     */
    public LoggerOutputConfigBuilder setTargetExclusions(String[] targetExclusions) {
        this.targetExclusions = targetExclusions;
        return this;
    }

    /**
     * Set color flag of an output.
     * 
     * @param colorEnabled Color flag of an output.
     * @return the builder
     */
    public LoggerOutputConfigBuilder setColorEnabled(boolean colorEnabled) {
        this.colorEnabled = colorEnabled;
        return this;
    }
}
