let config = {
    debug: true,
    limits: { max: 100, min: 0 },
    tags: ["a", "b", "c"],
};

println(config.limits);   // 100
println(config.tags[1]);      // b